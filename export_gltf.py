"""
A blender script for exporting automatically.

This script exports every collection inside the gltf-export collection into individual gltf files.
"""
import bpy

def export_gltf(file): 
    for obj in bpy.data.objects:
        obj.select_set(False)

    for collection in bpy.data.collections["gltf-export"].children:
        for obj in collection.all_objects:
            obj.select_set(True)

        bpy.ops.export_scene.gltf(
            filepath=bpy.path.abspath("//{}".format(collection.name)), 
            use_selection=True,  
            export_format="GLB", 
            export_lights=collection.name.find("light")!=-1,
            export_cameras=collection.name.find("camera")!=-1, 
        )

        for obj in collection.all_objects:
            obj.select_set(False)

bpy.app.handlers.save_post.append(export_gltf)
