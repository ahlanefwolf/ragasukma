import {
  Canvas,
  FabricObject,
  Polygon,
  ActiveSelection,
  Point,
  util,
} from 'fabric';
import polygonClipping, { Ring, Polygon as PcPolygon, MultiPolygon } from 'polygon-clipping';
import { applyShapeStrokeAlignment } from './drawing';

interface ObjectStyle {
  fill: any;
  stroke: any;
  strokeWidth: number;
  strokeAlign?: any;
  name: string;
}

/**
 * Menentukan apakah sebuah objek Fabric merupakan 2D closed-shape yang valid
 * untuk operasi kalkulasi irisan (Shape Builder & Union).
 * Garis, konektor, panah, teks, dan freehand drawing diabaikan.
 */
export function is2DClosedShape(obj: any): boolean {
  if (!obj) return false;
  if (obj.__isHatchPreview || obj.__isPort) return false;

  const objType = (obj.type || '').toLowerCase();
  const disallowedTypes = [
    'line',
    'connector',
    'arrow',
    'i-text',
    'text',
    'textbox',
    'path', // freehand brush drawing
    'image',
  ];

  if (disallowedTypes.includes(objType)) {
    return false;
  }

  // Jika berupa group, cek apakah group tersebut adalah connector atau arrow kustom
  if (obj.isConnector || obj.arrowHead) {
    return false;
  }

  return true;
}

/**
 * Mengonversi objek Fabric apa pun menjadi geometri poligon absolut (GeoJSON Polygon)
 * yang kompatibel dengan library polygon-clipping.
 */
function fabricObjectToGeom(obj: any): PcPolygon | null {
  if (!is2DClosedShape(obj)) return null;

  try {
    const matrix = typeof obj.calcTransformMatrix === 'function' ? obj.calcTransformMatrix() : null;
    const objType = (obj.type || '').toLowerCase();
    const pts: { x: number; y: number }[] = [];

    if (matrix) {
      if (objType === 'rect') {
        const w = (obj.width || 0) / 2;
        const h = (obj.height || 0) / 2;
        const rx = Math.min(Math.max(obj.rx || 0, 0), w);
        const ry = Math.min(Math.max(obj.ry || 0, 0), h);

        if (rx > 0 || ry > 0) {
          const segs = 8;
          // Top-right corner
          for (let i = 0; i <= segs; i++) {
            const a = -Math.PI / 2 + (i / segs) * (Math.PI / 2);
            pts.push(util.transformPoint(new Point(w - rx + rx * Math.cos(a), -h + ry + ry * Math.sin(a)), matrix));
          }
          // Bottom-right corner
          for (let i = 0; i <= segs; i++) {
            const a = 0 + (i / segs) * (Math.PI / 2);
            pts.push(util.transformPoint(new Point(w - rx + rx * Math.cos(a), h - ry + ry * Math.sin(a)), matrix));
          }
          // Bottom-left corner
          for (let i = 0; i <= segs; i++) {
            const a = Math.PI / 2 + (i / segs) * (Math.PI / 2);
            pts.push(util.transformPoint(new Point(-w + rx + rx * Math.cos(a), h - ry + ry * Math.sin(a)), matrix));
          }
          // Top-left corner
          for (let i = 0; i <= segs; i++) {
            const a = Math.PI + (i / segs) * (Math.PI / 2);
            pts.push(util.transformPoint(new Point(-w + rx + rx * Math.cos(a), -h + ry + ry * Math.sin(a)), matrix));
          }
        } else {
          pts.push(
            util.transformPoint(new Point(-w, -h), matrix),
            util.transformPoint(new Point(w, -h), matrix),
            util.transformPoint(new Point(w, h), matrix),
            util.transformPoint(new Point(-w, h), matrix)
          );
        }
      } else if (objType === 'circle') {
        const r = obj.radius || 0;
        const segs = 64;
        for (let i = 0; i < segs; i++) {
          const a = (i / segs) * 2 * Math.PI;
          pts.push(util.transformPoint(new Point(r * Math.cos(a), r * Math.sin(a)), matrix));
        }
      } else if (objType === 'polygon' && Array.isArray(obj.points)) {
        const po = obj.pathOffset || { x: 0, y: 0 };
        obj.points.forEach((p: any) => {
          pts.push(util.transformPoint(new Point(p.x - po.x, p.y - po.y), matrix));
        });
      } else if (objType === 'triangle') {
        const w = (obj.width || 0) / 2;
        const h = (obj.height || 0) / 2;
        pts.push(
          util.transformPoint(new Point(0, -h), matrix),
          util.transformPoint(new Point(w, h), matrix),
          util.transformPoint(new Point(-w, h), matrix)
        );
      } else {
        const coords = obj.getCoords ? obj.getCoords() : [];
        coords.forEach((c: any) => pts.push({ x: c.x, y: c.y }));
      }
    } else {
      const coords = obj.getCoords ? obj.getCoords() : [];
      coords.forEach((c: any) => pts.push({ x: c.x, y: c.y }));
    }

    if (pts.length < 3) return null;

    const ring: Ring = pts.map((p) => [p.x, p.y]);
    // Tutup ring poligon jika titik awal & akhir berbeda
    const first = ring[0];
    const last = ring[ring.length - 1];
    if (Math.hypot(first[0] - last[0], first[1] - last[1]) > 0.0001) {
      ring.push([first[0], first[1]]);
    }

    return [ring];
  } catch (err) {
    console.warn('[ShapeBuilder] Error extracting polygon geometry:', err);
    return null;
  }
}

/**
 * Membuat Fabric.Polygon baru dari sebuah ring koordinat poligon
 */
function createPolygonFromRing(
  ring: Ring,
  options: { fill?: any; stroke?: any; strokeWidth?: number; strokeAlign?: any; name?: string }
): Polygon | null {
  if (ring.length < 3) return null;

  // Hapus titik penutup duplikat di akhir ring
  let ptsCoords = ring;
  const first = ring[0];
  const last = ring[ring.length - 1];
  if (ring.length > 3 && Math.hypot(first[0] - last[0], first[1] - last[1]) < 0.0001) {
    ptsCoords = ring.slice(0, -1);
  }

  if (ptsCoords.length < 3) return null;

  const minX = Math.min(...ptsCoords.map((p) => p[0]));
  const minY = Math.min(...ptsCoords.map((p) => p[1]));

  // Koordinat relatif terhadap left & top (origin: 'left', 'top')
  const localPoints = ptsCoords.map((p) => new Point(p[0] - minX, p[1] - minY));

  const hasBorder = !!options.stroke && options.stroke !== 'transparent' && (options.strokeWidth || 0) > 0;
  const poly = new Polygon(localPoints, {
    left: minX,
    top: minY,
    originX: 'left',
    originY: 'top',
    fill: options.fill ?? '#7BDFF2',
    stroke: hasBorder ? options.stroke : '',
    strokeWidth: hasBorder ? (options.strokeWidth ?? 2) : 0,
    strokeUniform: true,
    strokeAlign: options.strokeAlign || 'center',
    selectable: true,
  } as any);

  applyShapeStrokeAlignment(poly, options.strokeAlign || 'center', hasBorder ? (options.strokeWidth ?? 2) : 0);

  (poly as any).id = 'frag_' + Math.random().toString(36).substring(2, 9);
  (poly as any).customName = options.name || 'Pecahan Objek';
  poly.setCoords();

  return poly;
}

export class ShapeBuilderEngine {
  /**
   * Mengambil semua objek tertutup 2D yang sedang terpilih (hanya closed shape yang valid)
   */
  public static getSelectedObjects(canvas: Canvas | null): FabricObject[] {
    if (!canvas) return [];
    let selected: FabricObject[] = canvas.getActiveObjects();
    if (selected.length < 2) {
      const activeObj: any = canvas.getActiveObject();
      if (activeObj && typeof activeObj.getObjects === 'function') {
        selected = activeObj.getObjects().slice();
      }
    }
    return selected.filter((o) => is2DClosedShape(o));
  }

  /**
   * Menghapus semua objek preview arsiran/hatch dari kanvas
   */
  public static clearHatchPreview(canvas: Canvas | null): void {
    if (!canvas) return;
    const hatchObjects = canvas.getObjects().filter((o: any) => o.__isHatchPreview);
    if (hatchObjects.length > 0) {
      hatchObjects.forEach((o) => canvas.remove(o));
      canvas.requestRenderAll();
    }
  }

  /**
   * Memperbarui arsiran preview irisan jika ada 2 atau lebih bentuk terpilih yang saling bertumpuk
   */
  public static updateHatchPreview(canvas: Canvas | null): void {
    if (!canvas) return;
    this.clearHatchPreview(canvas);

    const selected = this.getSelectedObjects(canvas);
    if (selected.length < 2) return;

    try {
      const geoms = selected.map((o) => fabricObjectToGeom(o)).filter(Boolean) as PcPolygon[];
      if (geoms.length < 2) return;

      // Cari irisan antara pasangan objek
      for (let i = 0; i < geoms.length; i++) {
        for (let j = i + 1; j < geoms.length; j++) {
          const inter = polygonClipping.intersection(geoms[i], geoms[j]);
          if (!inter || inter.length === 0) continue;

          inter.forEach((polyGeom) => {
            const outerRing = polyGeom[0];
            if (!outerRing || outerRing.length < 3) return;

            const previewObj = createPolygonFromRing(outerRing, {
              fill: 'rgba(245, 158, 11, 0.35)',
              stroke: '#F59E0B',
              strokeWidth: 1.5,
              name: 'Preview Irisan',
            });

            if (previewObj) {
              (previewObj as any).__isHatchPreview = true;
              previewObj.selectable = false;
              previewObj.evented = false;
              previewObj.strokeDashArray = [4, 4];
              canvas.add(previewObj);
            }
          });
        }
      }

      canvas.requestRenderAll();
    } catch (err) {
      console.warn('[ShapeBuilder] Error updating hatch preview:', err);
    }
  }

  /**
   * PISAHKAN SEMUA: Membelah semua objek yang bertumpuk menjadi potongan-potongan kecil
   * objek mandiri (Polygon) yang bisa dipilih, digeser, atau dihapus satu per satu.
   */
  public static splitIntoFragments(canvas: Canvas | null, onStateChange?: () => void): boolean {
    if (!canvas) return false;
    this.clearHatchPreview(canvas);

    const selected = this.getSelectedObjects(canvas);
    if (selected.length < 2) return false;

    const canvasObjects = canvas.getObjects();
    const sorted = [...selected].sort(
      (a, b) => canvasObjects.indexOf(a) - canvasObjects.indexOf(b)
    );

    // Kumpulkan style dan geometri untuk setiap objek terpilih
    interface ItemData {
      geom: PcPolygon | MultiPolygon;
      style: ObjectStyle;
    }

    const items: ItemData[] = [];
    sorted.forEach((obj: any, idx) => {
      const g = fabricObjectToGeom(obj);
      if (g) {
        const hasBorder = !!obj.stroke && obj.stroke !== 'transparent' && (obj.strokeWidth || 0) > 0;
        items.push({
          geom: g,
          style: {
            fill: (obj.fill && obj.fill !== 'transparent') ? obj.fill : (idx === 0 ? '#7BDFF2' : '#FF6B6B'),
            stroke: hasBorder ? obj.stroke : '',
            strokeWidth: hasBorder ? (obj.strokeWidth || 2) : 0,
            strokeAlign: obj.strokeAlign || 'center',
            name: obj.customName || `Objek ${idx + 1}`,
          },
        });
      }
    });

    if (items.length < 2) return false;

    // Lakukan partisi fragmentasi iteratif untuk membelah setiap irisan menjadi potongan unik
    let fragments: ItemData[] = [];

    for (let idx = 0; idx < items.length; idx++) {
      const item = items[idx];
      let toProcess: (PcPolygon | MultiPolygon)[] = [item.geom];
      const nextFragments: ItemData[] = [];

      for (const existing of fragments) {
        let existingGeom: PcPolygon | MultiPolygon = existing.geom;
        const newToProcess: (PcPolygon | MultiPolygon)[] = [];

        for (const cur of toProcess) {
          const inter = polygonClipping.intersection(existingGeom, cur);
          if (inter.length > 0) {
            const remExisting = polygonClipping.difference(existingGeom, cur);
            const remCur = polygonClipping.difference(cur, existingGeom);

            existingGeom = remExisting;
            if (remCur.length > 0) {
              newToProcess.push(remCur);
            }

            nextFragments.push({
              geom: inter,
              style: {
                fill: '#F59E0B', // Aksen oranye untuk irisan pertemuan
                stroke: existing.style.stroke || item.style.stroke,
                strokeWidth: existing.style.strokeWidth || item.style.strokeWidth,
                name: 'Irisan Pertemuan',
              },
            });
          } else {
            newToProcess.push(cur);
          }
        }

        if (existingGeom.length > 0) {
          nextFragments.push({
            geom: existingGeom,
            style: existing.style,
          });
        }

        toProcess = newToProcess;
      }

      for (const rem of toProcess) {
        if (rem.length > 0) {
          nextFragments.push({
            geom: rem,
            style: item.style,
          });
        }
      }

      fragments = nextFragments;
    }

    if (fragments.length === 0) return false;

    // Konversi semua fragmen poligon menjadi Fabric.Polygon
    const createdObjects: FabricObject[] = [];

    fragments.forEach((frag) => {
      // frag.geom adalah MultiPolygon
      const multiPoly = frag.geom as MultiPolygon;
      multiPoly.forEach((polyGeom) => {
        const outerRing = polyGeom[0];
        if (!outerRing || outerRing.length < 3) return;

        const obj = createPolygonFromRing(outerRing, frag.style);
        if (obj) {
          createdObjects.push(obj);
        }
      });
    });

    if (createdObjects.length === 0) return false;

    // Hapus objek-objek lama yang dipotong
    canvas.discardActiveObject();
    sorted.forEach((o) => canvas.remove(o));

    // Masukkan pecahan-pecahan baru ke kanvas
    createdObjects.forEach((o) => canvas.add(o));

    // Seleksi seluruh pecahan agar pengguna bisa langsung melihat dan memanipulasinya
    const activeSel = new ActiveSelection(createdObjects, { canvas });
    canvas.setActiveObject(activeSel);
    canvas.requestRenderAll();

    onStateChange?.();
    return true;
  }

  /**
   * UNION / GABUNG ULANG: Meleburkan objek-objek yang terpilih menjadi bentuk poligon utuh.
   */
  public static unionSelected(canvas: Canvas | null, onStateChange?: () => void): boolean {
    if (!canvas) return false;
    this.clearHatchPreview(canvas);

    const selected = this.getSelectedObjects(canvas);
    if (selected.length < 2) return false;

    const geoms: PcPolygon[] = [];
    selected.forEach((obj) => {
      const g = fabricObjectToGeom(obj);
      if (g) geoms.push(g);
    });

    if (geoms.length < 2) return false;

    try {
      const united = polygonClipping.union(geoms[0], ...geoms.slice(1));
      if (!united || united.length === 0) return false;

      const firstObj = selected[0];
      const hasBorder = !!firstObj.stroke && firstObj.stroke !== 'transparent' && (firstObj.strokeWidth || 0) > 0;
      const style: ObjectStyle = {
        fill: (firstObj.fill && firstObj.fill !== 'transparent') ? firstObj.fill : '#7BDFF2',
        stroke: hasBorder ? firstObj.stroke : '',
        strokeWidth: hasBorder ? (firstObj.strokeWidth || 2) : 0,
        strokeAlign: (firstObj as any).strokeAlign || 'center',
        name: 'Gabungan Poligon (Union)',
      };

      const createdObjects: FabricObject[] = [];
      united.forEach((polyGeom) => {
        const outerRing = polyGeom[0];
        if (outerRing && outerRing.length >= 3) {
          const obj = createPolygonFromRing(outerRing, style);
          if (obj) createdObjects.push(obj);
        }
      });

      if (createdObjects.length === 0) return false;

      canvas.discardActiveObject();
      selected.forEach((o) => canvas.remove(o));

      createdObjects.forEach((o) => canvas.add(o));

      if (createdObjects.length === 1) {
        canvas.setActiveObject(createdObjects[0]);
      } else {
        const sel = new ActiveSelection(createdObjects, { canvas });
        canvas.setActiveObject(sel);
      }

      canvas.requestRenderAll();
      onStateChange?.();
      return true;
    } catch (err) {
      console.warn('[ShapeBuilder] Error uniting polygons:', err);
      return false;
    }
  }
}
